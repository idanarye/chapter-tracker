(ns chapter-tracker.view.single-episode-panel (:gen-class)
  (:use clojure.java.shell)
  (:use chapter-tracker.view.tools)
  (:use chapter-tracker.model)
)

(import
  '(java.awt GridBagConstraints Dimension)
  '(javax.swing JLabel JTextField)
  '(javax.swing.event DocumentListener)
)

(defn create-episode-panel-and-updating-function []
  (let [name-field          (JTextField. 10)
        volume-field        (JTextField. 5)
        episode-field       (JTextField. 5)
        file-field          (JTextField. 25)
        date-of-read-field  (JTextField. 5)
        episode-record-atom (atom nil)
        update-list-function-atom (atom nil)
       ]
    [(create-panel {:width 500 :height 200} ;episode panel

                   (add-with-constraints (JLabel. "Name:")
                                         (gridx 0) (gridy 0) (fill GridBagConstraints/BOTH))
                   (add-with-constraints name-field
                                         (gridx 1) (gridy 0) (fill GridBagConstraints/BOTH))

                   (add-with-constraints (JLabel. "Volume:")
                                         (gridx 2) (gridy 0) (fill GridBagConstraints/BOTH))
                   (add-with-constraints volume-field
                                         (gridx 3) (gridy 0) (fill GridBagConstraints/BOTH))

                   (add-with-constraints (JLabel. "Episode:")
                                         (gridx 4) (gridy 0) (fill GridBagConstraints/BOTH))
                   (add-with-constraints episode-field
                                         (gridx 5) (gridy 0) (fill GridBagConstraints/BOTH))

                   (add-with-constraints (JLabel. "File:")
                                         (gridx 0) (gridy 1) (fill GridBagConstraints/BOTH))
                   (add-with-constraints file-field
                                         (gridx 1) (gridy 1) (gridwidth 5) (fill GridBagConstraints/BOTH))
                   (add-with-constraints (action-button "..."
                                                        (if-let [file (choose-file)]
                                                          (.setText file-field file)))
                                         (gridx 4) (gridy 1) (fill GridBagConstraints/BOTH))

                   (add-with-constraints (action-button "OPEN"
                                                          (future (sh (-> @episode-record-atom :series :media :program) (.getText file-field)))
                                         ) (gridx 0) (gridy 2) (gridwidth 5) (fill GridBagConstraints/BOTH))

                   (add-with-constraints (JLabel. "Date of Read:")
                                         (gridx 0) (gridy 3) (gridwidth 4) (fill GridBagConstraints/BOTH))
                   (add-with-constraints date-of-read-field
                                         (gridx 3) (gridy 3) (gridwidth 4) (fill GridBagConstraints/BOTH))
                   (add-with-constraints (action-button "NOW"
                                                        (let [now              (java.util.Date.)
                                                              date-formatter   (java.text.SimpleDateFormat. "YYYY-MM-dd HH:mm:ss")
                                                              date-string      (.format date-formatter now)]
                                                          (.setText date-of-read-field date-string)
                                                          (update-episode (:episode-id @episode-record-atom)
                                                                          {:date_of_read date-string})
                                                          (@update-list-function-atom :date-of-read date-string)
                                                        )
                                         ) (gridx 2) (gridy 3) (fill GridBagConstraints/BOTH))

                   (doseq [[gui-field column-name db-field] [[volume-field          :volume-number    :volume      ]
                                                             [episode-field         :episode-number   :number      ]
                                                             [name-field            :episode-name     :name        ]
                                                             [file-field            :episode-file     :file        ]
                                                             [date-of-read-field    :date-of-read     :date_of_read]
                                                            ]]
                     (let [on-text-change (fn []
                                            (when @episode-record-atom
                                              (let [new-value (.getText gui-field)]
                                                (future (Thread/sleep 1000)
                                                  (let [gui-value (.getText gui-field)]
                                                    (when (= new-value gui-value)
                                                      (update-episode (:episode-id @episode-record-atom)
                                                                      {db-field new-value})
                                                      (@update-list-function-atom column-name new-value)
                                                    )
                                                  )
                                                )
                                              )
                                            )
                                          )]
                       (.. gui-field getDocument (addDocumentListener (proxy [DocumentListener] []
                                                                        (changeUpdate [e] (on-text-change))
                                                                        (insertUpdate [e] (on-text-change))
                                                                        (removeUpdate [e] (on-text-change))
                                                                      )))
                     )
                   )

     ) (fn [episode-record update-list-function] ;updating function
         (reset! episode-record-atom nil)
         (reset! update-list-function-atom update-list-function)
         (doseq [[gui-field record-field] {volume-field          :volume-number
                                           episode-field         :episode-number
                                           name-field            :episode-name
                                           file-field            :episode-file
                                           date-of-read-field    :date-of-read
                                          }]
                 (.setText gui-field (-> episode-record (or {}) record-field (or "") .toString))
         )
         (reset! episode-record-atom episode-record)
       )]
  )
)
