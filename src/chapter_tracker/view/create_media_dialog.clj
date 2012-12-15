(ns chapter-tracker.view.create-media-dialog (:gen-class)
  (:use chapter-tracker.view.tools)
  (:use chapter-tracker.model)
)
(import
  '(java.awt GridBagLayout GridBagConstraints Dimension)
  '(javax.swing JFrame JLabel JTextField)
)

(defn create-create-media-dialog[]
  (create-frame {:title "Create Media"}
                (let [media-name-field  (JTextField. 10)
                      base-dir-field    (JTextField. 10)
                      file-types-field  (JTextField. 10)
                      program-field     (JTextField. 10)
                     ]
                  (.setLayout frame (GridBagLayout.))
                  (add-with-constraints (JLabel. "Media Name:") (gridx 0) (gridy 0))
                  (add-with-constraints media-name-field (gridx 1) (gridy 0) (gridwidth 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (JLabel. "Base Directory") (gridx 0) (gridy 1))
                  (add-with-constraints base-dir-field   (gridx 1) (gridy 1) (fill GridBagConstraints/BOTH))
                  (add-with-constraints (action-button "..."
                                                       (if-let [dir (choose-dir)]
                                                         (.setText base-dir-field dir))
                                        ) (gridx 2) (gridy 1) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (JLabel. "File Types") (gridx 0) (gridy 2))
                  (add-with-constraints file-types-field    (gridx 1) (gridy 2) (gridwidth 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (JLabel. "Program") (gridx 0) (gridy 3))
                  (add-with-constraints program-field    (gridx 1) (gridy 3) (gridwidth 2) (fill GridBagConstraints/BOTH))

                  (add-with-constraints (action-button "SAVE"
                                                             (create-media (.getText media-name-field)
                                                                           (.getText base-dir-field)
                                                                           (.getText file-types-field)
                                                                           (.getText program-field))
                                                             (.dispose frame)
                                              )
                                        (gridx 0) (gridy 4) (gridwidth 3) (fill GridBagConstraints/BOTH))
                )
  )
)
