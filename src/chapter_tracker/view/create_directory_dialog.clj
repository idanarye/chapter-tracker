(ns chapter-tracker.view.create-directory-dialog
  (:require clojure.string)
  (:use chapter-tracker.view.tools)
  (:use chapter-tracker.model)
)
(import
  '(java.awt GridBagLayout GridBagConstraints Dimension)
  '(javax.swing JFrame JLabel JTextField)
)

(defn create-create-directory-dialog[series-record on-close-function]
  (create-frame {:title (str "Create Directory For " (.toString series-record))}
                (let [dir-field    (JTextField. 20)
                      pattern-field  (JTextField. (clojure.string/replace (:series-name series-record) #"\s" ".*"))
                     ]
                  (add-with-constraints (JLabel. "Directory") (gridx 0) (gridy 1))
                  (add-with-constraints dir-field   (gridx 1) (gridy 0) (fill GridBagConstraints/BOTH))
                  (add-with-constraints (action-button "..."
                                                       (if-let [dir (choose-dir (-> series-record :media :base-dir))]
                                                         (.setText dir-field dir))
                                        ) (gridx 2) (gridy 0) (fill GridBagConstraints/BOTH))

                  (.setLayout frame (GridBagLayout.))
                  (add-with-constraints (JLabel. "Pattern:") (gridx 1) (gridy 0))
                  (add-with-constraints pattern-field (gridx 1) (gridy 1) (gridwidth 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (action-button "SAVE"
                                                       (create-directory series-record
                                                                         (.getText dir-field)
                                                                         (.getText pattern-field))
                                                       (.dispose frame)
                                                       (on-close-function)
                                              )
                                        (gridx 0) (gridy 2) (gridwidth 3) (fill GridBagConstraints/BOTH))
                )
  )
)
